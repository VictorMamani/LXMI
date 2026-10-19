import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useRef,
  useState,
  type ReactNode,
} from "react";
import { Toast, type ToastTone } from "../components/Toast";

type ToastMessage = { id: number; text: string; tone: ToastTone };
type ShowToast = (text: string, tone?: ToastTone) => void;

const ToastContext = createContext<ShowToast | null>(null);
const VISIBLE_MS = 2600;

/** One short notice at a time, bottom-right (e.g. 'SPACE CH·02 LOADED'). */
export function ToastProvider({ children }: { children: ReactNode }) {
  const [message, setMessage] = useState<ToastMessage | null>(null);
  const counter = useRef(0);

  const show = useCallback<ShowToast>((text, tone = "info") => {
    setMessage({ id: ++counter.current, text, tone });
  }, []);

  useEffect(() => {
    if (!message) return;
    const timer = window.setTimeout(() => setMessage(null), VISIBLE_MS);
    return () => window.clearTimeout(timer);
  }, [message]);

  return (
    <ToastContext.Provider value={show}>
      {children}
      {message && (
        <Toast key={message.id} text={message.text} tone={message.tone} />
      )}
    </ToastContext.Provider>
  );
}

export function useToast(): ShowToast {
  const value = useContext(ToastContext);
  if (!value) throw new Error("useToast requires ToastProvider");
  return value;
}
