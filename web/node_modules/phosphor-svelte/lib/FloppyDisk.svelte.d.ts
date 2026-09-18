import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FloppyDiskIcon` instead.
 *
 * @example
 * ```svelte
 * <FloppyDisk color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FloppyDisk: Component<IconComponentProps, {}, "">;
type FloppyDisk = ReturnType<typeof FloppyDisk>;
export default FloppyDisk;
