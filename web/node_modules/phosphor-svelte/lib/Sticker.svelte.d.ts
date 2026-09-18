import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `StickerIcon` instead.
 *
 * @example
 * ```svelte
 * <Sticker color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Sticker: Component<IconComponentProps, {}, "">;
type Sticker = ReturnType<typeof Sticker>;
export default Sticker;
