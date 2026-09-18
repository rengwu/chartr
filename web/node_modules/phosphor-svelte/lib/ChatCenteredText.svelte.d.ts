import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ChatCenteredTextIcon` instead.
 *
 * @example
 * ```svelte
 * <ChatCenteredText color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ChatCenteredText: Component<IconComponentProps, {}, "">;
type ChatCenteredText = ReturnType<typeof ChatCenteredText>;
export default ChatCenteredText;
