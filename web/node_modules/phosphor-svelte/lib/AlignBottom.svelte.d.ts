import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `AlignBottomIcon` instead.
 *
 * @example
 * ```svelte
 * <AlignBottom color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const AlignBottom: Component<IconComponentProps, {}, "">;
type AlignBottom = ReturnType<typeof AlignBottom>;
export default AlignBottom;
