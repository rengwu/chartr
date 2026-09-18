import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <CodesandboxLogoIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CodesandboxLogoIcon: Component<IconComponentProps, {}, "">;
type CodesandboxLogoIcon = ReturnType<typeof CodesandboxLogoIcon>;
export default CodesandboxLogoIcon;
