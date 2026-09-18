import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `NotificationIcon` instead.
 *
 * @example
 * ```svelte
 * <Notification color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Notification: Component<IconComponentProps, {}, "">;
type Notification = ReturnType<typeof Notification>;
export default Notification;
