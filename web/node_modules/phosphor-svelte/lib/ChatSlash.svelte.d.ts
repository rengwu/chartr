import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ChatSlashIcon` instead.
 *
 * @example
 * ```svelte
 * <ChatSlash color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ChatSlash: Component<IconComponentProps, {}, "">;
type ChatSlash = ReturnType<typeof ChatSlash>;
export default ChatSlash;
