import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ChatCircleDotsIcon` instead.
 *
 * @example
 * ```svelte
 * <ChatCircleDots color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ChatCircleDots: Component<IconComponentProps, {}, "">;
type ChatCircleDots = ReturnType<typeof ChatCircleDots>;
export default ChatCircleDots;
