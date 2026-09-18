import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BowlSteamIcon` instead.
 *
 * @example
 * ```svelte
 * <BowlSteam color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BowlSteam: Component<IconComponentProps, {}, "">;
type BowlSteam = ReturnType<typeof BowlSteam>;
export default BowlSteam;
