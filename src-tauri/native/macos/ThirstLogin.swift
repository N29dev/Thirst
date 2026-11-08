// Native Thirst sign-in helper for macOS.
// Independently implemented for Thirst. Its design is informed by the documented
// browser sign-in behavior of AppleBlox, but no GPL-licensed source is copied.
//
// Security boundary: official Roblox sign-in -> non-persistent WKWebView ->
// HttpOnly cookie store -> *temporary* macOS Keychain entry.
// The cookie is never printed, passed via a process argument, or sent to JS.
// The parent Rust process validates the session with Roblox before promoting
// the pending credential to the permanent keychain entry.

import AppKit
import WebKit
import Security
import Foundation
import Darwin

private let service = "dev.n29.thirst.roblox"
private let pendingAccount = "pending-login"
private let maxCookieLength = 8192

private func trustedHost(_ url: URL) -> Bool {
    guard url.scheme?.lowercased() == "https",
          url.user == nil, url.password == nil,
          let host = url.host?.lowercased() else { return false }
    return host == "roblox.com" || host.hasSuffix(".roblox.com")
}

private func allowedFrameHost(_ url: URL) -> Bool {
    if trustedHost(url) { return true }
    guard url.scheme?.lowercased() == "https",
          url.user == nil, url.password == nil,
          let host = url.host?.lowercased() else { return false }
    let providers = [
        "arkoselabs.com", "funcaptcha.com",
        "hcaptcha.com", "recaptcha.net",
        "google.com", "gstatic.com", "googleapis.com"
    ]
    return providers.contains { host == $0 || host.hasSuffix("." + $0) }
}

private func validSessionCookie(_ cookie: HTTPCookie) -> Bool {
    let domain = cookie.domain.lowercased().trimmingCharacters(in: CharacterSet(charactersIn: "."))
    return cookie.name == ".ROBLOSECURITY"
        && (domain == "roblox.com" || domain.hasSuffix(".roblox.com"))
        && cookie.isSecure
        && cookie.isHTTPOnly
        && !cookie.value.isEmpty
        && cookie.value.utf8.count <= maxCookieLength
}

private func storePendingCookie(_ cookie: String) -> Bool {
    // The pending entry is isolated from the existing primary credential.
    // Cancelling a new login never replaces an already connected account.
    let selector: [String: Any] = [
        kSecClass as String: kSecClassGenericPassword,
        kSecAttrService as String: service,
        kSecAttrAccount as String: pendingAccount
    ]
    SecItemDelete(selector as CFDictionary)

    guard let data = cookie.data(using: .utf8) else { return false }
    var record = selector
    record[kSecValueData as String] = data
    record[kSecAttrAccessible as String] = kSecAttrAccessibleWhenUnlockedThisDeviceOnly
    record[kSecAttrSynchronizable as String] = kCFBooleanFalse
    return SecItemAdd(record as CFDictionary, nil) == errSecSuccess
}

final class RobloxLoginWindow: NSObject, NSApplicationDelegate, NSWindowDelegate, WKNavigationDelegate {
    private var window: NSWindow!
    private var browser: WKWebView!
    private var sessionTimer: Timer?
    private var lifetimeTimer: Timer?
    private var parentTimer: Timer?
    private var finished = false
    private var readingCookies = false
    private var parentPID: pid_t = 0

    func applicationDidFinishLaunching(_ notification: Notification) {
        parentPID = ProcessInfo.processInfo.arguments.dropFirst().first
            .flatMap { Int32($0) } ?? getppid()
        if parentPID <= 1 { terminate("LOGIN_ERROR"); return }

        let config = WKWebViewConfiguration()
        config.websiteDataStore = .nonPersistent()

        browser = WKWebView(frame: .zero, configuration: config)
        browser.navigationDelegate = self
        browser.customUserAgent = nil // Leave Safari/WebKit's normal UA intact.
        browser.translatesAutoresizingMaskIntoConstraints = false

        let root = NSView(frame: NSRect(x: 0, y: 0, width: 540, height: 740))
        let header = NSView()
        header.translatesAutoresizingMaskIntoConstraints = false

        let urlIndicator = NSTextField(labelWithString: "🔒  https://www.roblox.com/login")
        urlIndicator.textColor = NSColor.secondaryLabelColor
        urlIndicator.font = NSFont.systemFont(ofSize: 12)
        urlIndicator.translatesAutoresizingMaskIntoConstraints = false
        header.addSubview(urlIndicator)

        let reload = NSButton(title: "Reload", target: self, action: #selector(reloadPage))
        reload.translatesAutoresizingMaskIntoConstraints = false
        header.addSubview(reload)

        root.addSubview(header)
        root.addSubview(browser)
        NSLayoutConstraint.activate([
            header.leadingAnchor.constraint(equalTo: root.leadingAnchor),
            header.trailingAnchor.constraint(equalTo: root.trailingAnchor),
            header.topAnchor.constraint(equalTo: root.topAnchor),
            header.heightAnchor.constraint(equalToConstant: 43),
            urlIndicator.leadingAnchor.constraint(equalTo: header.leadingAnchor, constant: 15),
            urlIndicator.centerYAnchor.constraint(equalTo: header.centerYAnchor),
            reload.trailingAnchor.constraint(equalTo: header.trailingAnchor, constant: -12),
            reload.centerYAnchor.constraint(equalTo: header.centerYAnchor),
            browser.topAnchor.constraint(equalTo: header.bottomAnchor),
            browser.leadingAnchor.constraint(equalTo: root.leadingAnchor),
            browser.trailingAnchor.constraint(equalTo: root.trailingAnchor),
            browser.bottomAnchor.constraint(equalTo: root.bottomAnchor)
        ])

        window = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: 540, height: 740),
            styleMask: [.titled, .closable, .miniaturizable, .resizable],
            backing: .buffered,
            defer: false
        )
        window.contentView = root
        window.title = "Official Roblox Login — Thirst"
        window.center()
        window.delegate = self
        window.makeKeyAndOrderFront(nil)
        NSApp.activate(ignoringOtherApps: true)

        guard let url = URL(string: "https://www.roblox.com/login") else {
            terminate("LOGIN_ERROR")
            return
        }
        browser.load(URLRequest(url: url, cachePolicy: .reloadIgnoringLocalCacheData))

        sessionTimer = Timer.scheduledTimer(timeInterval: 1.5, target: self,
                                            selector: #selector(checkSession),
                                            userInfo: nil, repeats: true)
        lifetimeTimer = Timer.scheduledTimer(timeInterval: 300, target: self,
                                             selector: #selector(timeout),
                                             userInfo: nil, repeats: false)
        parentTimer = Timer.scheduledTimer(timeInterval: 3, target: self,
                                           selector: #selector(checkParent),
                                           userInfo: nil, repeats: true)
    }

    @objc private func reloadPage() {
        if !finished { browser.reload() }
    }

    @objc private func checkParent() {
        if kill(parentPID, 0) != 0 {
            terminate("LOGIN_CANCELLED")
        }
    }

    @objc private func timeout() {
        terminate("LOGIN_TIMEOUT")
    }

    @objc private func checkSession() {
        guard !finished && !readingCookies else { return }
        readingCookies = true
        browser.configuration.websiteDataStore.httpCookieStore.getAllCookies { [weak self] cookies in
            guard let self = self else { return }
            self.readingCookies = false
            guard !self.finished else { return }
            guard let session = cookies.first(where: validSessionCookie) else { return }
            // No cookie text goes to stdout, stderr, window titles or WebKit scripts.
            if storePendingCookie(session.value) {
                self.terminate("LOGIN_STORED")
            } else {
                self.terminate("LOGIN_ERROR")
            }
        }
    }

    private func terminate(_ result: String) {
        guard !finished else { return }
        finished = true
        sessionTimer?.invalidate()
        lifetimeTimer?.invalidate()
        parentTimer?.invalidate()

        // Only a fixed status word is printed; stdout is never a secret channel.
        // The native parent observes the exit code instead of polling Keychain.
        // 0 = a pending session was stored, 1 = cancelled / timed out / failed.
        print(result)
        fflush(stdout)
        Darwin.exit(result == "LOGIN_STORED" ? EXIT_SUCCESS : EXIT_FAILURE)
    }

    func windowWillClose(_ notification: Notification) {
        terminate("LOGIN_CANCELLED")
    }

    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
        true
    }

    func webView(_ webView: WKWebView, decidePolicyFor action: WKNavigationAction,
                 decisionHandler: @escaping (WKNavigationActionPolicy) -> Void) {
        guard let url = action.request.url else {
            decisionHandler(.cancel); return
        }
        if url.absoluteString == "about:blank" {
            decisionHandler(.allow); return
        }
        if action.targetFrame?.isMainFrame == true {
            decisionHandler(trustedHost(url) ? .allow : .cancel)
        } else {
            // Roblox's security challenges may load a third-party iframe.
            // This never authorizes the launcher to send credentials there.
            decisionHandler(allowedFrameHost(url) ? .allow : .cancel)
        }
    }
}

let app = NSApplication.shared
app.setActivationPolicy(.regular)
let delegate = RobloxLoginWindow()
app.delegate = delegate
app.run()
