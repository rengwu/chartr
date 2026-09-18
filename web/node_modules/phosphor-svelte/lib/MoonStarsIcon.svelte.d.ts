import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <MoonStarsIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const MoonStarsIcon: Component<IconComponentProps, {}, "">;
type MoonStarsIcon = ReturnType<typeof MoonStarsIcon>;
export default MoonStarsIcon;
