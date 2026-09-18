import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FlashlightIcon` instead.
 *
 * @example
 * ```svelte
 * <Flashlight color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Flashlight: Component<IconComponentProps, {}, "">;
type Flashlight = ReturnType<typeof Flashlight>;
export default Flashlight;
