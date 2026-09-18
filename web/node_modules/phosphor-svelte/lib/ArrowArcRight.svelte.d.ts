import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowArcRightIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowArcRight color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowArcRight: Component<IconComponentProps, {}, "">;
type ArrowArcRight = ReturnType<typeof ArrowArcRight>;
export default ArrowArcRight;
