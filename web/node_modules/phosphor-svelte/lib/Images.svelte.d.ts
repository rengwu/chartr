import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ImagesIcon` instead.
 *
 * @example
 * ```svelte
 * <Images color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Images: Component<IconComponentProps, {}, "">;
type Images = ReturnType<typeof Images>;
export default Images;
