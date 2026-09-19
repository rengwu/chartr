import * as TabsPrimitive from '@radix-ui/react-tabs'
import * as React from 'react'
import { cn } from '@/lib/utils'

export const Tabs = TabsPrimitive.Root
export const TabsContent = React.forwardRef<React.ElementRef<typeof TabsPrimitive.Content>, React.ComponentPropsWithoutRef<typeof TabsPrimitive.Content>>(
  ({ className, ...props }, ref) => <TabsPrimitive.Content ref={ref} className={cn('outline-none', className)} {...props} />,
)
TabsContent.displayName = TabsPrimitive.Content.displayName
export const TabsList = React.forwardRef<React.ElementRef<typeof TabsPrimitive.List>, React.ComponentPropsWithoutRef<typeof TabsPrimitive.List>>(
  ({ className, ...props }, ref) => <TabsPrimitive.List ref={ref} className={cn('inline-flex h-9 items-center rounded-md bg-muted p-1 text-muted-foreground', className)} {...props} />,
)
TabsList.displayName = TabsPrimitive.List.displayName
export const TabsTrigger = React.forwardRef<React.ElementRef<typeof TabsPrimitive.Trigger>, React.ComponentPropsWithoutRef<typeof TabsPrimitive.Trigger>>(
  ({ className, ...props }, ref) => <TabsPrimitive.Trigger ref={ref} className={cn('inline-flex h-7 items-center justify-center rounded-sm px-3 text-xs font-medium outline-none transition-all focus-visible:ring-2 focus-visible:ring-ring data-[state=active]:bg-background data-[state=active]:text-foreground data-[state=active]:shadow-sm', className)} {...props} />,
)
TabsTrigger.displayName = TabsPrimitive.Trigger.displayName
