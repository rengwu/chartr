import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ChatTeardropTextIcon` instead.
 *
 * @example
 * ```svelte
 * <ChatTeardropText color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ChatTeardropText: Component<IconComponentProps, {}, "">;
type ChatTeardropText = ReturnType<typeof ChatTeardropText>;
export default ChatTeardropText;
