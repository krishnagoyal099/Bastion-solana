import React, { useState } from 'react';

const copyToClipboard = (text) => {
  navigator.clipboard.writeText(text);
};

const InstallSnippet = () => {
  const [activeTab, setActiveTab] = useState('linux');
  const [copied, setCopied] = useState(false);

  const commands = {
    linux: ['curl -fsSL https://raw.githubusercontent.com/krishnagoyal099/Bastion-solana/main/install-cli.sh | bash'],
    macos: ['curl -fsSL https://raw.githubusercontent.com/krishnagoyal099/Bastion-solana/main/install-cli.sh | bash'],
    windows: [
      'wsl --install',
      'curl -fsSL https://raw.githubusercontent.com/krishnagoyal099/Bastion-solana/main/install-cli.sh | bash'
    ]
  };

  const handleCopy = () => {
    copyToClipboard(commands[activeTab].join('\n'));
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="inline-block mt-8 border border-swiss-white/20 bg-swiss-black/60 backdrop-blur-md">
      <div className="flex border-b border-swiss-white/10">
        {['linux', 'macos', 'windows'].map(os => (
          <button 
            key={os}
            className={`px-6 py-3 text-xs font-swiss font-bold uppercase tracking-widest transition-colors ${activeTab === os ? 'bg-swiss-white text-swiss-black' : 'text-neutral-500 hover:text-white'}`}
            onClick={() => setActiveTab(os)}
          >
            {os}
          </button>
        ))}
      </div>
      <div 
        className="flex items-start gap-4 p-4 cursor-pointer group hover:bg-white/5 transition-colors" 
        onClick={handleCopy}
      >
        <div className="flex-1">
          {commands[activeTab].map((cmd, i) => (
            <code key={i} className="block font-mono text-xs text-swiss-white whitespace-nowrap mb-1 last:mb-0">
              <span className="text-swiss-accent mr-2">$</span>{cmd}
            </code>
          ))}
        </div>
        <span className="shrink-0 text-xs font-bold uppercase tracking-widest text-swiss-accent opacity-50 group-hover:opacity-100 transition-opacity">
          {copied ? 'COPIED!' : 'COPY'}
        </span>
      </div>
    </div>
  );
};

export default InstallSnippet;
