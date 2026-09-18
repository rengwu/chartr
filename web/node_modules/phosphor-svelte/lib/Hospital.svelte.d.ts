import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `HospitalIcon` instead.
 *
 * @example
 * ```svelte
 * <Hospital color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Hospital: Component<IconComponentProps, {}, "">;
type Hospital = ReturnType<typeof Hospital>;
export default Hospital;
