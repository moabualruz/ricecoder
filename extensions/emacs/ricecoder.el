;;; ricecoder.el --- RiceCoder IDE Integration for Emacs
;;
;; Author: RiceCoder Team
;; Version: 0.1.0
;; Package-Requires: ((emacs "26.1") (request "0.3.0") (json "1.5"))
;; Keywords: completion, diagnostics, lsp, ide
;; URL: https://github.com/ricecoder/ricecoder
;;
;;; Commentary:
;;
;; RiceCoder IDE integration for Emacs provides:
;; - Code completion via company-mode
;; - Diagnostics via flycheck
;; - Hover information
;; - Go to definition
;;
;;; Code:

(require 'json)
(require 'request)

;; Configuration
(defgroup ricecoder nil
  "RiceCoder IDE integration for Emacs."
  :group 'tools
  :prefix "ricecoder-")

(defcustom ricecoder-host "localhost"
  "RiceCoder backend host."
  :type 'string
  :group 'ricecoder)

(defcustom ricecoder-port 9000
  "RiceCoder backend port."
  :type 'integer
  :group 'ricecoder)

(defcustom ricecoder-timeout 5000
  "RiceCoder request timeout in milliseconds."
  :type 'integer
  :group 'ricecoder)

(defcustom ricecoder-enabled t
  "Enable RiceCoder integration."
  :type 'boolean
  :group 'ricecoder)

(defcustom ricecoder-debug nil
  "Enable debug logging."
  :type 'boolean
  :group 'ricecoder)

;; Internal variables
(defvar ricecoder-rpc-id 1
  "JSON-RPC request ID counter.")

(defvar ricecoder-pending-requests (make-hash-table :test 'equal)
  "Hash table of pending async requests.")

;; JSON-RPC Communication

(defun ricecoder-rpc-call (method params)
  "Send a synchronous JSON-RPC call to RiceCoder.
METHOD is the RPC method name.
PARAMS is the parameters object."
  (let* ((request-id ricecoder-rpc-id)
         (ricecoder-rpc-id (1+ ricecoder-rpc-id))
         (request-data (json-encode
                        `((jsonrpc . "2.0")
                          (id . ,request-id)
                          (method . ,method)
                          (params . ,params))))
         (url (format "http://%s:%d/rpc" ricecoder-host ricecoder-port))
         (response nil))
    
    (condition-case err
        (let ((response-data
               (request url
                 :type "POST"
                 :headers '(("Content-Type" . "application/json"))
                 :data request-data
                 :parser 'json-read
                 :sync t)))
          (if (plist-get response-data :data)
              (let ((result (plist-get response-data :data)))
                (if (plist-get result :result)
                    (plist-get result :result)
                  (if (plist-get result :error)
                      (ricecoder-error "RPC Error: %s" (plist-get (plist-get result :error) :message))
                    nil)))
            nil))
      (error
       (ricecoder-error "RPC Connection Error: %s" (error-message-string err))
       nil))))

(defun ricecoder-rpc-call-async (method params callback)
  "Send an asynchronous JSON-RPC call to RiceCoder.
METHOD is the RPC method name.
PARAMS is the parameters object.
CALLBACK is called with the response."
  (let* ((request-id ricecoder-rpc-id)
         (ricecoder-rpc-id (1+ ricecoder-rpc-id))
         (request-data (json-encode
                        `((jsonrpc . "2.0")
                          (id . ,request-id)
                          (method . ,method)
                          (params . ,params))))
         (url (format "http://%s:%d/rpc" ricecoder-host ricecoder-port)))
    
    (puthash request-id callback ricecoder-pending-requests)
    
    (request url
      :type "POST"
      :headers '(("Content-Type" . "application/json"))
      :data request-data
      :parser 'json-read
      :success (cl-function
                (lambda (&key data &allow-other-keys)
                  (let ((cb (gethash request-id ricecoder-pending-requests)))
                    (when cb
                      (if (plist-get data :result)
                          (funcall cb (plist-get data :result))
                        (if (plist-get data :error)
                            (ricecoder-error "RPC Error: %s" (plist-get (plist-get data :error) :message))))
                      (remhash request-id ricecoder-pending-requests)))))
      :error (cl-function
              (lambda (&key error-thrown &allow-other-keys)
                (ricecoder-error "RPC Error: %s" (error-message-string error-thrown))
                (remhash request-id ricecoder-pending-requests))))))

;; Completion

(defun ricecoder-get-completions ()
  "Get completions at point."
  (let* ((language (or (cdr (assoc major-mode ricecoder-language-map)) "text"))
         (file-path (buffer-file-name))
         (line (1- (line-number-at-pos)))
         (column (1- (current-column)))
         (params `((language . ,language)
                   (file_path . ,file-path)
                   (position . ((line . ,line)
                               (character . ,column)))
                   (context . ,(thing-at-point 'line)))))
    
    (ricecoder-rpc-call "completion/get_completions" params)))

(defun ricecoder-completion-at-point ()
  "Completion at point function for completion-at-point-functions."
  (let* ((bounds (bounds-of-thing-at-point 'symbol))
         (start (or (car bounds) (point)))
         (end (or (cdr bounds) (point)))
         (completions (ricecoder-get-completions)))
    
    (when completions
      (list start end
            (mapcar (lambda (item)
                      (plist-get item :label))
                    completions)))))

;; Diagnostics

(defun ricecoder-get-diagnostics ()
  "Get diagnostics for current buffer."
  (let* ((language (or (cdr (assoc major-mode ricecoder-language-map)) "text"))
         (file-path (buffer-file-name))
         (source (buffer-string))
         (params `((language . ,language)
                   (file_path . ,file-path)
                   (source . ,source))))
    
    (ricecoder-rpc-call "diagnostics/get_diagnostics" params)))

(defun ricecoder-update-diagnostics ()
  "Update diagnostics for current buffer."
  (when ricecoder-enabled
    (let ((diagnostics (ricecoder-get-diagnostics)))
      (when diagnostics
        (ricecoder-display-diagnostics diagnostics)))))

(defun ricecoder-display-diagnostics (diagnostics)
  "Display diagnostics in the current buffer."
  (dolist (diagnostic diagnostics)
    (let* ((line (1+ (plist-get (plist-get (plist-get diagnostic :range) :start) :line)))
           (column (plist-get (plist-get (plist-get diagnostic :range) :start) :character))
           (severity (plist-get diagnostic :severity))
           (message (plist-get diagnostic :message)))
      
      (when (and line column)
        (save-excursion
          (goto-line line)
          (move-to-column column)
          (ricecoder-display-diagnostic-at-point message severity))))))

(defun ricecoder-display-diagnostic-at-point (message severity)
  "Display a diagnostic message at point."
  (let ((overlay (make-overlay (point) (1+ (point)))))
    (overlay-put overlay 'face (if (= severity 1) 'error 'warning))
    (overlay-put overlay 'help-echo message)))

;; Hover

(defun ricecoder-get-hover ()
  "Get hover information at point."
  (let* ((language (or (cdr (assoc major-mode ricecoder-language-map)) "text"))
         (file-path (buffer-file-name))
         (line (1- (line-number-at-pos)))
         (column (1- (current-column)))
         (params `((language . ,language)
                   (file_path . ,file-path)
                   (position . ((line . ,line)
                               (character . ,column))))))
    
    (ricecoder-rpc-call "hover/get_hover" params)))

(defun ricecoder-show-hover ()
  "Show hover information at point."
  (interactive)
  (let ((hover (ricecoder-get-hover)))
    (when hover
      (let ((contents (plist-get hover :contents)))
        (message "%s" contents)))))

;; Go to Definition

(defun ricecoder-get-definition ()
  "Get definition location at point."
  (let* ((language (or (cdr (assoc major-mode ricecoder-language-map)) "text"))
         (file-path (buffer-file-name))
         (line (1- (line-number-at-pos)))
         (column (1- (current-column)))
         (params `((language . ,language)
                   (file_path . ,file-path)
                   (position . ((line . ,line)
                               (character . ,column))))))
    
    (ricecoder-rpc-call "definition/get_definition" params)))

(defun ricecoder-goto-definition ()
  "Go to definition at point."
  (interactive)
  (let ((definition (ricecoder-get-definition)))
    (when definition
      (let* ((file-path (plist-get definition :file_path))
             (line (1+ (plist-get (plist-get (plist-get definition :range) :start) :line)))
             (column (plist-get (plist-get (plist-get definition :range) :start) :character)))
        
        (find-file file-path)
        (goto-line line)
        (move-to-column column)))))

(defun ricecoder-goto-definition-other-window ()
  "Go to definition in other window."
  (interactive)
  (let ((definition (ricecoder-get-definition)))
    (when definition
      (let* ((file-path (plist-get definition :file_path))
             (line (1+ (plist-get (plist-get (plist-get definition :range) :start) :line)))
             (column (plist-get (plist-get (plist-get definition :range) :start) :character)))
        
        (find-file-other-window file-path)
        (goto-line line)
        (move-to-column column)))))

;; Error Handling

(defun ricecoder-error (format-string &rest args)
  "Report an error to the user."
  (message (concat "RiceCoder Error: " (apply 'format format-string args))))

(defun ricecoder-warn (format-string &rest args)
  "Report a warning to the user."
  (message (concat "RiceCoder Warning: " (apply 'format format-string args))))

(defun ricecoder-info (format-string &rest args)
  "Report info to the user."
  (message (concat "RiceCoder: " (apply 'format format-string args))))

(defun ricecoder-debug (format-string &rest args)
  "Log a debug message."
  (when ricecoder-debug
    (message (concat "RiceCoder DEBUG: " (apply 'format format-string args)))))

;; Language Mapping

(defvar ricecoder-language-map
  '((rust-mode . "rust")
    (rustic-mode . "rust")
    (typescript-mode . "typescript")
    (js-mode . "javascript")
    (js2-mode . "javascript")
    (python-mode . "python")
    (python-ts-mode . "python")
    (c-mode . "c")
    (c++-mode . "cpp")
    (java-mode . "java")
    (go-mode . "go")
    (ruby-mode . "ruby")
    (php-mode . "php"))
  "Mapping from Emacs major modes to language identifiers.")

;; Mode Setup

(defun ricecoder-setup ()
  "Set up RiceCoder integration for current buffer."
  (when ricecoder-enabled
    ;; Add completion function
    (add-hook 'completion-at-point-functions 'ricecoder-completion-at-point nil t)
    
    ;; Set up diagnostics update on change
    (add-hook 'after-change-functions (lambda (&rest _) (ricecoder-update-diagnostics)) nil t)
    
    ;; Set up keybindings
    (local-set-key (kbd "C-c C-d") 'ricecoder-goto-definition)
    (local-set-key (kbd "C-c C-o") 'ricecoder-goto-definition-other-window)
    (local-set-key (kbd "C-c C-h") 'ricecoder-show-hover)))

;;;###autoload
(define-minor-mode ricecoder-mode
  "RiceCoder IDE integration mode."
  :lighter " RiceCoder"
  :keymap (let ((map (make-sparse-keymap)))
            (define-key map (kbd "C-c C-d") 'ricecoder-goto-definition)
            (define-key map (kbd "C-c C-o") 'ricecoder-goto-definition-other-window)
            (define-key map (kbd "C-c C-h") 'ricecoder-show-hover)
            map)
  
  (if ricecoder-mode
      (ricecoder-setup)
    (progn
      ;; Clean up when mode is disabled
      (remove-hook 'completion-at-point-functions 'ricecoder-completion-at-point t)
      (remove-hook 'after-change-functions (lambda (&rest _) (ricecoder-update-diagnostics)) t))))

;;;###autoload
(define-globalized-minor-mode global-ricecoder-mode
  ricecoder-mode
  (lambda ()
    (when (and ricecoder-enabled
               (not (minibufferp)))
      (ricecoder-mode 1)))
  :group 'ricecoder)

(provide 'ricecoder)
;;; ricecoder.el ends here
