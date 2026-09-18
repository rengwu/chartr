import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TelegramLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <TelegramLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TelegramLogo: Component<IconComponentProps, {}, "">;
type TelegramLogo = ReturnType<typeof TelegramLogo>;
export default TelegramLogo;
