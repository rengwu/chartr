import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BuildingOfficeIcon` instead.
 *
 * @example
 * ```svelte
 * <BuildingOffice color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BuildingOffice: Component<IconComponentProps, {}, "">;
type BuildingOffice = ReturnType<typeof BuildingOffice>;
export default BuildingOffice;
