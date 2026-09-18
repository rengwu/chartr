import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `DoorOpenIcon` instead.
 *
 * @example
 * ```svelte
 * <DoorOpen color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const DoorOpen: Component<IconComponentProps, {}, "">;
type DoorOpen = ReturnType<typeof DoorOpen>;
export default DoorOpen;
