import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowElbowLeftUpIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowElbowLeftUp color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowElbowLeftUp: Component<IconComponentProps, {}, "">;
type ArrowElbowLeftUp = ReturnType<typeof ArrowElbowLeftUp>;
export default ArrowElbowLeftUp;
