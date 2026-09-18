import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ChatTeardropDotsIcon` instead.
 *
 * @example
 * ```svelte
 * <ChatTeardropDots color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ChatTeardropDots: Component<IconComponentProps, {}, "">;
type ChatTeardropDots = ReturnType<typeof ChatTeardropDots>;
export default ChatTeardropDots;
