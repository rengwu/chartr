import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ChatTeardropIcon` instead.
 *
 * @example
 * ```svelte
 * <ChatTeardrop color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ChatTeardrop: Component<IconComponentProps, {}, "">;
type ChatTeardrop = ReturnType<typeof ChatTeardrop>;
export default ChatTeardrop;
