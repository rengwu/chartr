import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CastleTurretIcon` instead.
 *
 * @example
 * ```svelte
 * <CastleTurret color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CastleTurret: Component<IconComponentProps, {}, "">;
type CastleTurret = ReturnType<typeof CastleTurret>;
export default CastleTurret;
