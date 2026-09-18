import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SquaresFourIcon` instead.
 *
 * @example
 * ```svelte
 * <SquaresFour color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SquaresFour: Component<IconComponentProps, {}, "">;
type SquaresFour = ReturnType<typeof SquaresFour>;
export default SquaresFour;
