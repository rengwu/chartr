import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowULeftUpIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowULeftUp color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowULeftUp: Component<IconComponentProps, {}, "">;
type ArrowULeftUp = ReturnType<typeof ArrowULeftUp>;
export default ArrowULeftUp;
