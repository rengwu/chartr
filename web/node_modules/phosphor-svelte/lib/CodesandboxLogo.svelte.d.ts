import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CodesandboxLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <CodesandboxLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CodesandboxLogo: Component<IconComponentProps, {}, "">;
type CodesandboxLogo = ReturnType<typeof CodesandboxLogo>;
export default CodesandboxLogo;
