import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <TelegramLogoIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TelegramLogoIcon: Component<IconComponentProps, {}, "">;
type TelegramLogoIcon = ReturnType<typeof TelegramLogoIcon>;
export default TelegramLogoIcon;
