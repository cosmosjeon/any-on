import { useMemo, useEffect, useState } from 'react';
import Form from '@rjsf/core';
import { RJSFValidationError } from '@rjsf/utils';
import validator from '@rjsf/validator-ajv8';

import { Alert, AlertDescription } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { Loader2 } from 'lucide-react';
import { shadcnTheme } from './rjsf';
// Using custom shadcn/ui widgets instead of @rjsf/shadcn theme

type ExecutorType =
  | 'AMP'
  | 'CLAUDE_CODE'
  | 'GEMINI'
  | 'CODEX'
  | 'CURSOR_AGENT'
  | 'COPILOT'
  | 'OPENCODE'
  | 'QWEN_CODE';

// ExecutorConfig represents the configuration data for an executor
// It's a flexible object that conforms to the executor's JSON schema
// Using 'any' here because RJSF library requires it for form data
// eslint-disable-next-line @typescript-eslint/no-explicit-any
type ExecutorConfig = any;

interface ExecutorConfigFormProps {
  executor: ExecutorType;
  value: ExecutorConfig;
  onSubmit?: (formData: ExecutorConfig) => void;
  onChange?: (formData: ExecutorConfig) => void;
  onSave?: (formData: ExecutorConfig) => Promise<void>;
  disabled?: boolean;
  isSaving?: boolean;
  isDirty?: boolean;
}

import schemas from 'virtual:executor-schemas';

export function ExecutorConfigForm({
  executor,
  value,
  onSubmit,
  onChange,
  onSave,
  disabled = false,
  isSaving = false,
  isDirty = false,
}: ExecutorConfigFormProps) {
  const [formData, setFormData] = useState(value || {});
  const [validationErrors, setValidationErrors] = useState<
    RJSFValidationError[]
  >([]);

  const schema = useMemo(() => {
    return schemas[executor];
  }, [executor]);

  useEffect(() => {
    setFormData(value || {});
    setValidationErrors([]);
  }, [value, executor]);

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const handleChange = ({ formData: newFormData }: any) => {
    setFormData(newFormData);
    if (onChange) {
      onChange(newFormData);
    }
  };

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const handleSubmit = async ({ formData: submitData }: any) => {
    setValidationErrors([]);
    if (onSave) {
      await onSave(submitData);
    } else if (onSubmit) {
      onSubmit(submitData);
    }
  };

  const handleError = (errors: RJSFValidationError[]) => {
    setValidationErrors(errors);
  };

  if (!schema) {
    return (
      <Alert variant="destructive">
        <AlertDescription>
          Schema not found for executor type: {executor}
        </AlertDescription>
      </Alert>
    );
  }

  return (
    <div className="space-y-6">
      <div className="rounded-lg border bg-card shadow-sm">
        <div className="p-6">
          <Form
            schema={schema}
            formData={formData}
            onChange={handleChange}
            onSubmit={handleSubmit}
            onError={handleError}
            validator={validator}
            disabled={disabled}
            liveValidate
            showErrorList={false}
            widgets={shadcnTheme.widgets}
            templates={shadcnTheme.templates}
          >
            <div className="hidden">
              <button type="submit" />
            </div>
          </Form>
        </div>
        {onSave && (
          <div className="flex justify-end px-6 py-4 border-t bg-muted/30">
            <Button
              onClick={() => handleSubmit({ formData } as any)}
              disabled={!isDirty || validationErrors.length > 0 || isSaving}
            >
              {isSaving && (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              )}
              Save Configuration
            </Button>
          </div>
        )}
      </div>

      {validationErrors.length > 0 && (
        <Alert variant="destructive">
          <AlertDescription>
            <ul className="list-disc list-inside space-y-1">
              {validationErrors.map((error, index) => (
                <li key={index}>
                  {error.property}: {error.message}
                </li>
              ))}
            </ul>
          </AlertDescription>
        </Alert>
      )}
    </div>
  );
}
