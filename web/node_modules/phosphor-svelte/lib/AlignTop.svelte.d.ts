import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `AlignTopIcon` instead.
 *
 * @example
 * ```svelte
 * <AlignTop color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const AlignTop: Component<IconComponentProps, {}, "">;
type AlignTop = ReturnType<typeof AlignTop>;
export default AlignTop;
