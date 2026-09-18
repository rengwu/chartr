import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowBendLeftUpIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowBendLeftUp color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowBendLeftUp: Component<IconComponentProps, {}, "">;
type ArrowBendLeftUp = ReturnType<typeof ArrowBendLeftUp>;
export default ArrowBendLeftUp;
