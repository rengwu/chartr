import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `LightbulbFilamentIcon` instead.
 *
 * @example
 * ```svelte
 * <LightbulbFilament color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const LightbulbFilament: Component<IconComponentProps, {}, "">;
type LightbulbFilament = ReturnType<typeof LightbulbFilament>;
export default LightbulbFilament;
