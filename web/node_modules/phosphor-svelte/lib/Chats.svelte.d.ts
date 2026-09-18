import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ChatsIcon` instead.
 *
 * @example
 * ```svelte
 * <Chats color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Chats: Component<IconComponentProps, {}, "">;
type Chats = ReturnType<typeof Chats>;
export default Chats;
