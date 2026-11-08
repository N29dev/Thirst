import { isValidServerId } from "./servers";

// This is NOT automatic geolocation and is never uploaded. Users can annotate
// a short-lived Roblox instance after checking where it actually hosted.
export const REGION_OPTIONS = [
  { code: "NA", label: "North America" },
  { code: "EU", label: "Europe" },
  { code: "SA", label: "South America" },
  { code: "AS", label: "Asia" },
  { code: "OC", label: "Oceania" },
  { code: "ME", label: "Middle East" },
  { code: "AF", label: "Africa" }
] as const;

export type RegionCode = (typeof REGION_OPTIONS)[number]["code"];
export type RegionFilter = RegionCode | "any" | "unknown";
export type LocalRegionReport = {
  placeId: number;
  serverId: string;
  region: RegionCode;
  observedAt: number;
  source: "local-report";
};

// Roblox server UUIDs are temporary. These observations expire rather than
// making a historical server ID look like a verified, currently live machine.
export const REGION_REPORT_TTL_MS = 2 * 60 * 60 * 1000;
const MAX_REPORTS = 200;
const STORAGE_KEY = "thirst.region-reports.v1";

export function isRegionCode(value: unknown): value is RegionCode {
  return REGION_OPTIONS.some(option => option.code === value);
}

export function regionLabel(code: RegionCode): string {
  return REGION_OPTIONS.find(option => option.code === code)?.label ?? "Unknown";
}

export function activeRegionReports(reports: LocalRegionReport[], now = Date.now()): LocalRegionReport[] {
  const seen = new Set<string>();
  const result: LocalRegionReport[] = [];
  for (const entry of reports) {
    if (!entry || !Number.isSafeInteger(entry.placeId) || entry.placeId <= 0 ||
        !isValidServerId(entry.serverId) || !isRegionCode(entry.region) ||
        !Number.isFinite(entry.observedAt) ||
        entry.observedAt > now + 60_000 || now - entry.observedAt >= REGION_REPORT_TTL_MS ||
        entry.source !== "local-report") continue;
    const key = entry.placeId + ":" + entry.serverId.toLowerCase();
    if (seen.has(key)) continue;
    seen.add(key);
    result.push({
      placeId: entry.placeId,
      serverId: entry.serverId.toLowerCase(),
      region: entry.region,
      observedAt: entry.observedAt,
      source: "local-report"
    });
    if (result.length >= MAX_REPORTS) break;
  }
  return result;
}

export function readRegionReports(): LocalRegionReport[] {
  try {
    const raw: unknown = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? "[]");
    return activeRegionReports(Array.isArray(raw) ? raw.slice(0, MAX_REPORTS * 2) as LocalRegionReport[] : []);
  } catch {
    return [];
  }
}

export function writeRegionReports(reports: LocalRegionReport[]): LocalRegionReport[] {
  const active = activeRegionReports(reports);
  localStorage.setItem(STORAGE_KEY, JSON.stringify(active));
  return active;
}

export function reportForServer(
  reports: LocalRegionReport[], placeId: number | null, serverId: string, now = Date.now()
): LocalRegionReport | null {
  if (!placeId) return null;
  const key = serverId.toLowerCase();
  return reports.find(entry =>
    entry.placeId === placeId &&
    entry.serverId === key &&
    now >= entry.observedAt &&
    now - entry.observedAt < REGION_REPORT_TTL_MS
  ) ?? null;
}

export function withRegionReport(
  reports: LocalRegionReport[], placeId: number, serverId: string, region: RegionCode
): LocalRegionReport[] {
  if (!Number.isSafeInteger(placeId) || placeId <= 0 ||
      !isValidServerId(serverId) || !isRegionCode(region)) return reports;
  const key = serverId.toLowerCase();
  return activeRegionReports([
    {
      placeId, serverId: key, region, observedAt: Date.now(), source: "local-report"
    },
    ...reports.filter(entry => !(entry.placeId === placeId && entry.serverId === key))
  ]);
}

export function withoutRegionReport(
  reports: LocalRegionReport[], placeId: number, serverId: string
): LocalRegionReport[] {
  return reports.filter(entry =>
    !(entry.placeId === placeId && entry.serverId === serverId.toLowerCase()));
}
