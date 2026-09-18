import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ChatCircleIcon` instead.
 *
 * @example
 * ```svelte
 * <ChatCircle color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ChatCircle: Component<IconComponentProps, {}, "">;
type ChatCircle = ReturnType<typeof ChatCircle>;
export default ChatCircle;
