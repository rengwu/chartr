import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SmileyStickerIcon` instead.
 *
 * @example
 * ```svelte
 * <SmileySticker color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SmileySticker: Component<IconComponentProps, {}, "">;
type SmileySticker = ReturnType<typeof SmileySticker>;
export default SmileySticker;
