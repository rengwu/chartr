import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `RocketLaunchIcon` instead.
 *
 * @example
 * ```svelte
 * <RocketLaunch color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const RocketLaunch: Component<IconComponentProps, {}, "">;
type RocketLaunch = ReturnType<typeof RocketLaunch>;
export default RocketLaunch;
