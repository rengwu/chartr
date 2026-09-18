import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ChatCenteredIcon` instead.
 *
 * @example
 * ```svelte
 * <ChatCentered color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ChatCentered: Component<IconComponentProps, {}, "">;
type ChatCentered = ReturnType<typeof ChatCentered>;
export default ChatCentered;
