import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `HardDrivesIcon` instead.
 *
 * @example
 * ```svelte
 * <HardDrives color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const HardDrives: Component<IconComponentProps, {}, "">;
type HardDrives = ReturnType<typeof HardDrives>;
export default HardDrives;
