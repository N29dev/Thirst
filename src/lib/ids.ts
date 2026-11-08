// Shared ID validation; separated to avoid cyclic dependencies between
// the server filter and region-lookup modules.
export const SERVER_UUID_PATTERN = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

export function isValidServerId(value: unknown): value is string {
  return typeof value === "string" && SERVER_UUID_PATTERN.test(value);
}
