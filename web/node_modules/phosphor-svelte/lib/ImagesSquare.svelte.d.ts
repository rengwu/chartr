import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ImagesSquareIcon` instead.
 *
 * @example
 * ```svelte
 * <ImagesSquare color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ImagesSquare: Component<IconComponentProps, {}, "">;
type ImagesSquare = ReturnType<typeof ImagesSquare>;
export default ImagesSquare;
