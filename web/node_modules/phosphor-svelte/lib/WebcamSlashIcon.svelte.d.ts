import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <WebcamSlashIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const WebcamSlashIcon: Component<IconComponentProps, {}, "">;
type WebcamSlashIcon = ReturnType<typeof WebcamSlashIcon>;
export default WebcamSlashIcon;
