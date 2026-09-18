import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ExcludeSquareIcon` instead.
 *
 * @example
 * ```svelte
 * <ExcludeSquare color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ExcludeSquare: Component<IconComponentProps, {}, "">;
type ExcludeSquare = ReturnType<typeof ExcludeSquare>;
export default ExcludeSquare;
