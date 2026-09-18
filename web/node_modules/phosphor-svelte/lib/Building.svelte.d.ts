import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BuildingIcon` instead.
 *
 * @example
 * ```svelte
 * <Building color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Building: Component<IconComponentProps, {}, "">;
type Building = ReturnType<typeof Building>;
export default Building;
