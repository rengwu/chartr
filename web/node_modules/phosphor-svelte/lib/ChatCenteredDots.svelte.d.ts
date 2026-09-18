import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ChatCenteredDotsIcon` instead.
 *
 * @example
 * ```svelte
 * <ChatCenteredDots color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ChatCenteredDots: Component<IconComponentProps, {}, "">;
type ChatCenteredDots = ReturnType<typeof ChatCenteredDots>;
export default ChatCenteredDots;
