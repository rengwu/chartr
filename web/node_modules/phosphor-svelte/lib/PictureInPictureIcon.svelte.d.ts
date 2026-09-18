import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <PictureInPictureIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PictureInPictureIcon: Component<IconComponentProps, {}, "">;
type PictureInPictureIcon = ReturnType<typeof PictureInPictureIcon>;
export default PictureInPictureIcon;
