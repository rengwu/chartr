import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FediverseLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <FediverseLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FediverseLogo: Component<IconComponentProps, {}, "">;
type FediverseLogo = ReturnType<typeof FediverseLogo>;
export default FediverseLogo;
