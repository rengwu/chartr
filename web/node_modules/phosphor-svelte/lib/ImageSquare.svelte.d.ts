import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ImageSquareIcon` instead.
 *
 * @example
 * ```svelte
 * <ImageSquare color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ImageSquare: Component<IconComponentProps, {}, "">;
type ImageSquare = ReturnType<typeof ImageSquare>;
export default ImageSquare;
