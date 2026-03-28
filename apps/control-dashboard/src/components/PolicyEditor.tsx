import { useState, useEffect, useCallback } from 'react'
import type { MachinePolicy, UpdatePolicyRequest } from '../api/policies'
import { ACTION_CATEGORIES, AUTONOMY_LEVELS, SESSION_MODES } from '../api/policies'
import { Button } from './ui/Button'
import { Select } from './ui/Select'
import { Toggle } from './ui/Toggle'

interface PolicyEditorProps {
  policy: MachinePolicy
  onSave: (data: UpdatePolicyRequest) => Promise<void>
  showSessionMode?: boolean
}

export function PolicyEditor({ policy, onSave, showSessionMode = true }: PolicyEditorProps) {
  const [allowedCategories, setAllowedCategories] = useState<Set<string>>(new Set())
  const [blockedCategories, setBlockedCategories] = useState<Set<string>>(new Set())
  const [maxAutonomyLevel, setMaxAutonomyLevel] = useState('normal')
  const [sessionMode, setSessionMode] = useState('normal')
  const [requireApprovalFor, setRequireApprovalFor] = useState<Set<string>>(new Set())
  const [saving, setSaving] = useState(false)
  const [saveError, setSaveError] = useState<string | null>(null)
  const [saveSuccess, setSaveSuccess] = useState(false)

  const resetFromPolicy = useCallback((p: MachinePolicy) => {
    setAllowedCategories(new Set(p.allowed_categories))
    setBlockedCategories(new Set(p.blocked_categories))
    setMaxAutonomyLevel(p.max_autonomy_level)
    setSessionMode(p.session_mode)
    setRequireApprovalFor(new Set(p.require_approval_for))
  }, [])

  useEffect(() => {
    resetFromPolicy(policy)
  }, [policy, resetFromPolicy])

  function toggleAllowed(category: string, checked: boolean) {
    setAllowedCategories((prev) => {
      const next = new Set(prev)
      if (checked) {
        next.add(category)
      } else {
        next.delete(category)
      }
      return next
    })
    // If allowing, remove from blocked
    if (checked) {
      setBlockedCategories((prev) => {
        const next = new Set(prev)
        next.delete(category)
        return next
      })
    }
  }

  function toggleBlocked(category: string, checked: boolean) {
    setBlockedCategories((prev) => {
      const next = new Set(prev)
      if (checked) {
        next.add(category)
      } else {
        next.delete(category)
      }
      return next
    })
    // If blocking, remove from allowed
    if (checked) {
      setAllowedCategories((prev) => {
        const next = new Set(prev)
        next.delete(category)
        return next
      })
    }
  }

  function toggleApprovalRequired(category: string, checked: boolean) {
    setRequireApprovalFor((prev) => {
      const next = new Set(prev)
      if (checked) {
        next.add(category)
      } else {
        next.delete(category)
      }
      return next
    })
  }

  async function handleSave() {
    setSaving(true)
    setSaveError(null)
    setSaveSuccess(false)
    try {
      const data: UpdatePolicyRequest = {
        allowed_categories: Array.from(allowedCategories),
        blocked_categories: Array.from(blockedCategories),
        max_autonomy_level: maxAutonomyLevel,
        require_approval_for: Array.from(requireApprovalFor),
      }
      if (showSessionMode) {
        data.session_mode = sessionMode
      }
      await onSave(data)
      setSaveSuccess(true)
      setTimeout(() => setSaveSuccess(false), 3000)
    } catch (err) {
      setSaveError(err instanceof Error ? err.message : 'Failed to save policy')
    } finally {
      setSaving(false)
    }
  }

  return (
    <div className="space-y-6">
      {/* Action Categories */}
      <div>
        <h4 className="text-sm font-medium text-secondary-900">Action Categories</h4>
        <p className="mt-1 text-xs text-secondary-500">
          Configure which action categories are allowed or blocked.
        </p>
        <div className="mt-3 divide-y divide-secondary-100">
          {ACTION_CATEGORIES.map(({ value, label }) => {
            const isAllowed = allowedCategories.has(value)
            const isBlocked = blockedCategories.has(value)
            return (
              <div key={value} className="flex items-center justify-between py-2.5">
                <div>
                  <span className="text-sm font-medium text-secondary-900">{label}</span>
                  <span className="ml-2 text-xs text-secondary-400">{value}</span>
                </div>
                <div className="flex items-center gap-4">
                  <label className="flex items-center gap-1.5 text-xs">
                    <input
                      type="checkbox"
                      checked={isAllowed}
                      onChange={(e) => toggleAllowed(value, e.target.checked)}
                      className="h-4 w-4 rounded border-secondary-300 text-primary-600 focus:ring-primary-500"
                    />
                    <span className="text-success-700">Allow</span>
                  </label>
                  <label className="flex items-center gap-1.5 text-xs">
                    <input
                      type="checkbox"
                      checked={isBlocked}
                      onChange={(e) => toggleBlocked(value, e.target.checked)}
                      className="h-4 w-4 rounded border-secondary-300 text-danger-600 focus:ring-danger-500"
                    />
                    <span className="text-danger-700">Block</span>
                  </label>
                </div>
              </div>
            )
          })}
        </div>
      </div>

      {/* Max Autonomy Level */}
      <Select
        label="Max Autonomy Level"
        options={[...AUTONOMY_LEVELS]}
        value={maxAutonomyLevel}
        onChange={setMaxAutonomyLevel}
      />

      {/* Session Mode */}
      {showSessionMode && (
        <Select
          label="Session Mode"
          options={[...SESSION_MODES]}
          value={sessionMode}
          onChange={setSessionMode}
        />
      )}

      {/* Require Approval For */}
      <div>
        <h4 className="text-sm font-medium text-secondary-900">Require Approval For</h4>
        <p className="mt-1 text-xs text-secondary-500">
          Actions in these categories will require human approval before execution.
        </p>
        <div className="mt-3 space-y-2">
          {ACTION_CATEGORIES.map(({ value, label }) => (
            <Toggle
              key={value}
              label={label}
              checked={requireApprovalFor.has(value)}
              onChange={(checked) => toggleApprovalRequired(value, checked)}
            />
          ))}
        </div>
      </div>

      {/* Save */}
      <div className="flex items-center gap-3">
        <Button loading={saving} onClick={() => void handleSave()}>
          Save Policy
        </Button>
        {saveSuccess && (
          <span className="text-sm text-success-700">Policy saved successfully.</span>
        )}
        {saveError && (
          <span className="text-sm text-danger-700">{saveError}</span>
        )}
      </div>
    </div>
  )
}
