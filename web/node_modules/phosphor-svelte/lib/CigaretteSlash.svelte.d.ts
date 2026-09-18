import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CigaretteSlashIcon` instead.
 *
 * @example
 * ```svelte
 * <CigaretteSlash color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CigaretteSlash: Component<IconComponentProps, {}, "">;
type CigaretteSlash = ReturnType<typeof CigaretteSlash>;
export default CigaretteSlash;
