'use client';
import { X } from 'lucide-react';

interface SubmissionFormProps {
  onCancel: () => void;
  onSubmit: () => void;
}

export const SubmissionForm = ({ onCancel, onSubmit }: SubmissionFormProps) => {
  return (
    <div className="space-y-4 animate-in fade-in slide-in-from-bottom-4 duration-300">
      <div className="flex justify-between items-center mb-2">
        <h4 className="text-[10px] font-black uppercase text-emerald-500 tracking-widest">Protocol Upload</h4>
        <button onClick={onCancel} className="text-slate-500 hover:text-white transition-colors">
          <X size={16} />
        </button>
      </div>
      <input 
        className="w-full bg-white/5 border border-white/10 rounded-xl p-3 text-xs outline-none focus:border-emerald-500/50" 
        placeholder="Link to GitHub PR / Repository"
      />
      <textarea 
        className="w-full bg-white/5 border border-white/10 rounded-xl p-3 text-xs outline-none focus:border-emerald-500/50" 
        placeholder="Submission notes (optional)..."
        rows={3}
      />
      <button 
        onClick={onSubmit} 
        className="w-full py-4 bg-white text-black text-[10px] font-black uppercase tracking-widest rounded-xl hover:bg-emerald-400 transition-colors"
      >
        Confirm & Encrypt
      </button>
    </div>
  );
};