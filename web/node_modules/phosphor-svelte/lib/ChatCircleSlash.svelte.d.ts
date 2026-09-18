import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ChatCircleSlashIcon` instead.
 *
 * @example
 * ```svelte
 * <ChatCircleSlash color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ChatCircleSlash: Component<IconComponentProps, {}, "">;
type ChatCircleSlash = ReturnType<typeof ChatCircleSlash>;
export default ChatCircleSlash;
