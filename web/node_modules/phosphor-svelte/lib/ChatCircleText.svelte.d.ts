import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ChatCircleTextIcon` instead.
 *
 * @example
 * ```svelte
 * <ChatCircleText color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ChatCircleText: Component<IconComponentProps, {}, "">;
type ChatCircleText = ReturnType<typeof ChatCircleText>;
export default ChatCircleText;
