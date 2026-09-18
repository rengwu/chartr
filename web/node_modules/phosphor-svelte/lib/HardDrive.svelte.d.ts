import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `HardDriveIcon` instead.
 *
 * @example
 * ```svelte
 * <HardDrive color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const HardDrive: Component<IconComponentProps, {}, "">;
type HardDrive = ReturnType<typeof HardDrive>;
export default HardDrive;
