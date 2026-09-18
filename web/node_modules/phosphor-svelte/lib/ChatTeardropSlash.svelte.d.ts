import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ChatTeardropSlashIcon` instead.
 *
 * @example
 * ```svelte
 * <ChatTeardropSlash color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ChatTeardropSlash: Component<IconComponentProps, {}, "">;
type ChatTeardropSlash = ReturnType<typeof ChatTeardropSlash>;
export default ChatTeardropSlash;
