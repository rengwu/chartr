import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `UniteSquareIcon` instead.
 *
 * @example
 * ```svelte
 * <UniteSquare color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const UniteSquare: Component<IconComponentProps, {}, "">;
type UniteSquare = ReturnType<typeof UniteSquare>;
export default UniteSquare;
