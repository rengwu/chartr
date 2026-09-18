import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `RecycleIcon` instead.
 *
 * @example
 * ```svelte
 * <Recycle color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Recycle: Component<IconComponentProps, {}, "">;
type Recycle = ReturnType<typeof Recycle>;
export default Recycle;
