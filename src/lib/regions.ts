import { isValidServerId } from "./ids";

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

// RoValra's API provides reported city/region/country values for Roblox
// server UUIDs. The country-to-continent grouping below is our own UI mapping;
// unknown/ambiguous codes remain Unknown and are never inferred from ping.
export type ProviderRegion = {
  serverId: string;
  city: string | null;
  region: string | null;
  country: string | null;
  source: "rovalra";
  via?: "glutony" | "direct";
};

const COUNTRY_GROUPS: Record<RegionCode, string[]> = {
  NA: "US CA MX GT BZ HN SV NI CR PA CU DO HT JM BS BB TT AG DM GD KN LC VC PR".split(" "),
  EU: "AL AD AT BY BE BA BG HR CZ DK EE FI FR DE GR HU IS IE IT XK LV LI LT LU MT MD MC NL MK NO PL PT RO SM RS SK SI ES SE CH UA GB VA".split(" "),
  SA: "BR AR BO CL CO EC PY PE UY VE GY SR GF FK".split(" "),
  AS: "CN JP KR KP TW HK MO SG MY TH VN PH ID IN BD PK LK NP KH LA MM MN KZ UZ KG TJ TM AF BT MV BN TL AZ AM GE".split(" "),
  OC: "AU NZ FJ PG WS TO VU SB FM MH PW NR KI TV".split(" "),
  ME: "AE SA QA BH KW OM YE IR IQ IL JO LB SY TR PS".split(" "),
  AF: "DZ AO BJ BW BF BI CM CV CF TD KM CD CG CI DJ EG GQ ER ET GA GM GH GN GW KE LS LR LY MG MW ML MR MU MA MZ NA NE NG RW ST SN SC SL SO ZA SS SD SZ TZ TG TN UG ZM ZW".split(" ")
};

const COUNTRY_ALIASES: Record<string, string> = {
  "UNITED STATES": "US", "UNITED STATES OF AMERICA": "US", USA: "US",
  "UNITED KINGDOM": "GB", UK: "GB",
  "SOUTH KOREA": "KR", "REPUBLIC OF KOREA": "KR",
  "UNITED ARAB EMIRATES": "AE", "SAUDI ARABIA": "SA",
  "GERMANY": "DE", "FRANCE": "FR", "NETHERLANDS": "NL",
  "SINGAPORE": "SG", "BRAZIL": "BR", "AUSTRALIA": "AU",
  "JAPAN": "JP", "INDIA": "IN", "CANADA": "CA", "KAZAKHSTAN": "KZ",
  "MEXICO": "MX", "ARGENTINA": "AR", "CHILE": "CL", "COLOMBIA": "CO",
  "PERU": "PE", "URUGUAY": "UY", "PORTUGAL": "PT", "SPAIN": "ES",
  "ITALY": "IT", "SWEDEN": "SE", "NORWAY": "NO", "FINLAND": "FI",
  "DENMARK": "DK", "POLAND": "PL", "BELGIUM": "BE", "SWITZERLAND": "CH",
  "AUSTRIA": "AT", "IRELAND": "IE", "CZECH REPUBLIC": "CZ",
  "CZECHIA": "CZ", "ROMANIA": "RO", "TURKEY": "TR", "TÜRKIYE": "TR",
  "CHINA": "CN", "TAIWAN": "TW", "HONG KONG": "HK", "THAILAND": "TH",
  "VIETNAM": "VN", "PHILIPPINES": "PH", "INDONESIA": "ID",
  "MALAYSIA": "MY", "PAKISTAN": "PK", "BANGLADESH": "BD",
  "NEW ZEALAND": "NZ", "EGYPT": "EG", "SOUTH AFRICA": "ZA",
  "NIGERIA": "NG", "KENYA": "KE"
};

// Roblox and RoValra can return ISO codes or a country name. Convert only
// known country values; never invent a flag for an unrecognized location.
export function providerCountryCode(region: ProviderRegion | null | undefined): string | null {
  if (!region?.country) return null;
  const raw = region.country.trim().toUpperCase();
  const code = COUNTRY_ALIASES[raw] ?? raw;
  if (!/^[A-Z]{2}$/.test(code)) return null;
  return Object.values(COUNTRY_GROUPS).some(codes => codes.includes(code)) ? code : null;
}

export function providerCountryFlag(region: ProviderRegion | null | undefined): string {
  const code = providerCountryCode(region);
  if (!code) return "🌐";
  // Unicode regional indicator symbols. Uses the OS emoji font, not a
  // downloaded flag asset or a third-party image service.
  return String.fromCodePoint(
    0x1f1e6 + code.charCodeAt(0) - 65,
    0x1f1e6 + code.charCodeAt(1) - 65
  );
}

export function providerCompactRegionLabel(region: ProviderRegion | null | undefined): string {
  if (!region) return "Unknown";
  // City + country is much easier to scan than city + state + country in a
  // narrow server list; the complete location stays available on hover.
  const city = region.city?.trim();
  const area = region.region?.trim();
  const country = region.country?.trim();
  const main = city || area || country;
  if (!main) return "Unknown";
  return country && main.toLowerCase() !== country.toLowerCase()
    ? main + ", " + country
    : main;
}

export function providerBroadRegion(region: ProviderRegion | null | undefined): RegionCode | null {
  const country = providerCountryCode(region);
  if (!country) return null;
  for (const option of REGION_OPTIONS) {
    if (COUNTRY_GROUPS[option.code].includes(country)) return option.code;
  }
  return null;
}

export function providerRegionLabel(region: ProviderRegion | null | undefined): string {
  if (!region) return "Unknown";
  const location = [region.city, region.region, region.country]
    .filter((part): part is string => typeof part === "string" && part.trim().length > 0);
  const unique = [...new Set(location.map(item => item.trim()))];
  return unique.length ? unique.join(", ") : "Unknown";
}

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
