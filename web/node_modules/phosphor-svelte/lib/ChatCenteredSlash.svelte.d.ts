import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ChatCenteredSlashIcon` instead.
 *
 * @example
 * ```svelte
 * <ChatCenteredSlash color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ChatCenteredSlash: Component<IconComponentProps, {}, "">;
type ChatCenteredSlash = ReturnType<typeof ChatCenteredSlash>;
export default ChatCenteredSlash;
