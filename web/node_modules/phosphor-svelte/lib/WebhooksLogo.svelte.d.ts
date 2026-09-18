import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `WebhooksLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <WebhooksLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const WebhooksLogo: Component<IconComponentProps, {}, "">;
type WebhooksLogo = ReturnType<typeof WebhooksLogo>;
export default WebhooksLogo;
